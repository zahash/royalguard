<div align="center">

# royalguard

[![Crates.io](https://img.shields.io/crates/v/royalguard.svg)](https://crates.io/crates/royalguard)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

> Ergonomic Command Line Password Manager. Forever Free. Made with ❤️ using 🦀

## Download

[https://github.com/zahash/royalguard/releases](https://github.com/zahash/royalguard/releases)

## Usage examples

```sh
Add, Update:
    set gmail user = sussolini sensitive pass = 'use single quote for spaces' url = mail.google.sus
    set gmail sensitive pass = updatedpassword

Delete whole record: 
    del gmail

Delete fields: 
    del gmail url pass

Show -- replaces sensitive values with *****:
    show all
    show gmail
    show user is sussolini and (pass contains sus or url matches '.*com')

Show (filter by name):
    show $name contains mail
    show . contains mail

Reveal -- works exactly like Show but doesnt respect sensitivity
    reveal user is sussolini and (pass contains sus or url matches '.*com')

Copy field to clipboard:
    copy gmail pass
```

## Meta

M. Zahash – zahash.z@gmail.com

Distributed under the MIT license. See `LICENSE` for more information.

[https://github.com/zahash/](https://github.com/zahash/)

## Contributing

1. Fork it (<https://github.com/zahash/royalguard/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

