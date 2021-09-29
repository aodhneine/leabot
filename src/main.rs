mod tls;
mod base64;

mod log {
	#[inline]
	pub fn warn(args: std::fmt::Arguments<'_>) {
		eprintln!("\x1b[1;33mwarning:\x1b[0m {}", args);
	}

	#[inline]
	pub fn error(args: std::fmt::Arguments<'_>) {
		eprintln!("\x1b[1;31merror:\x1b[0m {}", args);
	}

	#[inline]
	pub fn fatal(args: std::fmt::Arguments<'_>) -> ! {
		eprintln!("\x1b[1;31mfatal:\x1b[0m {}", args);
		std::process::exit(1);
	}
}

mod config {
	use crate::log;

	#[derive(Debug)]
	pub struct Config<'a> {
		pub server: &'a str,
		pub api_key: &'a str,
		pub name: &'a str,
	}

	fn split_line<'a>(s: &'a str) -> (&'a str, &'a str) {
		return unsafe {
			// Position of the new line character.
			let i = s.find('\n').unwrap_or(s.len());
			( s.get_unchecked(..i), s.get(i + 1..).unwrap_or("") )
		};
	}

	fn string<'a>(s: &'a str) -> Option<&'a str> {
		if s.len() < 2 {
			return None;
		}
		
		let bytes = s.as_bytes();

		if bytes[0] == b'"' && bytes[s.len() - 1] == b'"' {			
			return Some(unsafe {
				s.get_unchecked(1..s.len() - 1)
			});
		}
		
		return None;
	}

	#[allow(unreachable_code)]
	pub fn parse<'a>(s: &'a str) -> Option<Config<'a>> {
		// We use MaybeUninit to allow us to easily initialise this struct field by
		// field without creating tons of local variables, or storing Options in the
		// config struct itself.
		let mut config = core::mem::MaybeUninit::<Config<'a>>::uninit();
		let ptr = config.as_mut_ptr();

		let mut s = s;

		loop {
			let (line, rest) = split_line(s);
			s = rest;

			// Ignore comments.
			if line.starts_with('#') {
				continue;
			}
			
			// Ignore empty lines.
			if line.trim().is_empty() {
				continue;
			}

			// Parse a single line.
			let (key, value) = line.split_at(match line.find('=') {
				Some(i) => i,
				_ => {
					log::error(format_args!("invalid key-value pair, expected '='"));
					return None;
				},
			});

			// Remove whitespace from around the '=' sign.
			let key = key.trim_end();
			let value = unsafe {
				value.get_unchecked(1..).trim_start()
			};

			match key {
				"server" => match string(value) {
					None => {
						log::error(format_args!("expected string as value for 'server'"));
						return None;
					},
					Some(s) => unsafe {
						core::ptr::addr_of_mut!((*ptr).server).write(s)
					},
				},
				"api_key" => match string(value) {
					None => {
						log::error(format_args!("expected string as value for 'api_key'"));
						return None;
					},
					Some(s) => unsafe {
						core::ptr::addr_of_mut!((*ptr).api_key).write(s)
					},
				},
				"name" => match string(value) {
					None => {
						log::error(format_args!("expected string as a value for 'name'"));
						return None;
					},
					Some(s) => unsafe {
						core::ptr::addr_of_mut!((*ptr).name).write(s)
					},
				},
				_ => log::warn(format_args!("unknown key: {}", key)),
			}

			if rest == "" {
				break;
			}
		}
		
		// TODO: Verify that actually all the fields are initialised.
		
		// SAFETY: We cannot reach this return without initialising all the fields
		// of Config, or otherwise we would return None sooner.
		return Some(unsafe {
			config.assume_init()
		});
	}
}

#[inline]
fn wipe(bytes: &mut [u8]) {
	bytes.fill(0);
}

fn main() {
	let config = match std::fs::read_to_string("config.toml") {
		Ok(s) => s,
		Err(_) => log::fatal(format_args!("could not find config file")),
	};

	let config = match config::parse(&config) {
		Some(c) => c,
		None => log::fatal(format_args!("failed to parse config file")),
	};

	let mut client = tls::Client::new();
	client.connect("leas-elia-diam.zulipchat.com:443");

	let mut auth_plain = format!("{}-bot@{}:{}", config.name, config.server, config.api_key);
	// We need to base64 encode the api key and the bot email.
	let mut auth_encoded = vec![0u8; base64::bound(auth_plain.len())];
	base64::encode(&mut auth_encoded, auth_plain.as_bytes());

	// Now we need to wipe the plain-text auth string to prevent malicious actors
	// from accessing it by snooping at the process memory.
	wipe(unsafe {
		auth_plain.as_bytes_mut()
	});
	drop(auth_plain);

	// We know that base64 encoding produces valid ASCII strings, which are just
	// valid UTF-8 strings.
	let auth = unsafe {
		std::str::from_utf8_unchecked(&auth_encoded)
	};

	// We need to url encode the message.
	let content = "type=stream&to=general&subject=test%20from%20rust&content=sixth%20message%20send%20by%20this%20bot%20from%20rust%20using%20the%20api";

	// TODO: Have something similar to hyper's Request type, which constructs
	// requests at runtime through a set of methods.
	let request= format!("POST /api/v1/messages HTTP/1.1\r\nHost:leas-elia-diam.zulipchat.com\r\nAuthorization: Basic {}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}\r\n\r\n", auth, content.len(), content);
	client.write(request.as_bytes()).unwrap();

	use std::io::Read;

	let mut buf = String::new();
	client.read_to_string(&mut buf).unwrap();
	print!("{}", buf);
}
