<div align="center">

# royalguard

[![Crates.io](https://img.shields.io/crates/v/royalguard.svg)](https://crates.io/crates/royalguard)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

> Ergonomic Command Line Password Manager. Forever Free.

## Download

[https://github.com/zahash/royalguard/releases](https://github.com/zahash/royalguard/releases)

## Usage examples

```sh
Add, Update and Delete:
    set gmail user = sussolini pass = 'use single quote for spaces' url = mail.google.sus
    set gmail pass = updatedpassword
    del gmail

Show:
    show all
    show gmail
    show user is sussolini and (pass contains sus or url matches '.*com')

Show (filter by name):
    show $name contains mail
    show . contains mail
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

