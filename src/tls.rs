use std::os::raw::c_void as void;

#[allow(non_camel_case_types)]
#[repr(C)]
struct tls_config {
	_unused: [u8; 0],
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct tls {
	_unused: [u8; 0],
}

#[allow(non_upper_case_globals)]
const TLS_PROTOCOL_TLSv1_2: u32 = 1 << 3;
#[allow(non_upper_case_globals)]
const TLS_PROTOCOL_TLSv1_3: u32 = 1 << 4;

const TLS_WANT_POLLIN: isize = -2;
const TLS_WANT_POLLOUT: isize = -3;

extern "C" {
	fn tls_config_new() -> *mut tls_config;
	fn tls_config_free(config: *mut tls_config);
	fn tls_config_error(config: *mut tls_config) -> *const i8;
	
	fn tls_config_set_protocols(config: *mut tls_config, protocols: u32) -> i32;

	fn tls_client() -> *mut tls;
	#[must_use]
	fn tls_configure(ctx: *mut tls, config: *mut tls_config) -> i32;
	fn tls_free(ctx: *mut tls);
	
	fn tls_connect(ctx: *mut tls, host: *const i8, port: *const i8) -> i32;

	fn tls_read(ctx: *mut tls, buf: *mut void, buflen: usize) -> isize;
	fn tls_write(ctx: *mut tls, buf: *const void, buflen: usize) -> isize;
	fn tls_close(ctx: *mut tls) -> i32;
	fn tls_error(ctx: *mut tls) -> *const i8;
}

pub struct Client {
	ctx: *mut tls,
}

impl Client {
	pub fn new() -> Self {
		let config = unsafe {
			tls_config_new()
		};

		if config.is_null() {
			panic!("failed to allocate tls config");
		}

		if unsafe {
			tls_config_set_protocols(config, TLS_PROTOCOL_TLSv1_3 | TLS_PROTOCOL_TLSv1_2)
		} != 0 {
			let error = unsafe {
				std::ffi::CStr::from_ptr(tls_config_error(config))
			}.to_str().unwrap();

			panic!("failed to configure tls: {}", error);
		}
		
		let ctx = unsafe {
			tls_client()
		};
		
		if ctx.is_null() {
			panic!("failed to allocate tls client");
		}

		if unsafe {
			tls_configure(ctx, config)
		} != 0 {
			panic!("failed to configure tls client");
		}
		
		// We don't need this config anymore, so we can deallocate it. Fortunately
		// libtls allows us to do it.
		unsafe {
			tls_config_free(config)
		};

		return Self {
			ctx,
		};
	}
	
	pub fn connect(&mut self, addr: &str) {
		let (host, port) = addr.split_at(addr.find(':').unwrap());
		let port = unsafe {
			port.get_unchecked(1..)
		};
		let (host, port) = {
			( std::ffi::CString::new(host).unwrap(), std::ffi::CString::new(port).unwrap() )
		};

		if unsafe {
			tls_connect(self.ctx, host.as_ptr(), port.as_ptr())
		} != 0 {
			let error = unsafe {
				std::ffi::CStr::from_ptr(tls_error(self.ctx))
			}.to_str().unwrap();

			panic!("failed to connect to {}: {}", addr, error);
		}
	}

	pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
		let mut bytes_written;

		loop {
			bytes_written = unsafe {
				tls_write(self.ctx, buf.as_ptr() as *mut void, buf.len())
			};
	
			if bytes_written == TLS_WANT_POLLOUT || bytes_written == TLS_WANT_POLLIN {
				continue;
			}
			
			if bytes_written == -1 {
				let error = unsafe {
					std::ffi::CStr::from_ptr(tls_error(self.ctx))
				}.to_str().unwrap();
	
				eprintln!("failed to write data: {}", error);
				return None;
			}

			break;
		}
		
		return Some(bytes_written as usize);
	}
}

impl std::io::Read for Client {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let mut bytes_read;

		loop {
			bytes_read = unsafe {
				tls_read(self.ctx, buf.as_mut_ptr() as *mut void, buf.len())
			};

			if bytes_read == TLS_WANT_POLLIN || bytes_read == TLS_WANT_POLLOUT {
				continue;
			}

			if bytes_read == -1 {
				let error = unsafe {
					std::ffi::CStr::from_ptr(tls_error(self.ctx))
				}.to_str().unwrap();

				eprintln!("failed to read data: {}", error);
				return Err(std::io::Error::from(std::io::ErrorKind::Other));
			}

			break;
		}

		return Ok(bytes_read as usize);
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		unsafe {
			tls_close(self.ctx);
			tls_free(self.ctx);
		};
	}
}