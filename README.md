<div align="center">

<pre>
██████╗  ██████╗ ██╗   ██╗ █████╗ ██╗          ██████╗ ██╗   ██╗ █████╗ ██████╗ ██████╗ 
██╔══██╗██╔═══██╗╚██╗ ██╔╝██╔══██╗██║         ██╔════╝ ██║   ██║██╔══██╗██╔══██╗██╔══██╗
██████╔╝██║   ██║ ╚████╔╝ ███████║██║         ██║  ███╗██║   ██║███████║██████╔╝██║  ██║
██╔══██╗██║   ██║  ╚██╔╝  ██╔══██║██║         ██║   ██║██║   ██║██╔══██║██╔══██╗██║  ██║
██║  ██║╚██████╔╝   ██║   ██║  ██║███████╗    ╚██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝
╚═╝  ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ 
----------------------------------------------------------------------------------------
Secure Ergonomic Command Line Password Manager. Free Forever. Made with ❤️ using 🦀
</pre>

[![Crates.io](https://img.shields.io/crates/v/royalguard.svg)](https://crates.io/crates/royalguard)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

## 🚀 Download

[https://github.com/zahash/royalguard/releases](https://github.com/zahash/royalguard/releases)

( or )

```
cargo install royalguard
```

## 🧑‍💻 Unleash the Commands!

```sh
✨ Add, Update:
    set gmail user = sussolini sensitive pass = 'use single quote for spaces' url = mail.google.sus
    set gmail sensitive pass = updatedpassword user = updated_user

🔥 Delete whole record: 
    del gmail

🔥 Delete fields: 
    del gmail url pass

🔍 Show -- replaces sensitive values with *****:
    show all
    show gmail
    show user is sussolini and (pass contains sus or url matches '.*com')

🔍 Show (filter by name):
    show . contains gmail

🕵️ Reveal -- works exactly like Show but does not respect sensitivity
    reveal user is sussolini and (pass contains sus or url matches '.*com')

📜 History -- show changes made overtime:
    history gmail
    reveal history gmail

🖊️ Rename:
    rename gmail gmail2

📋 Copy field to clipboard:
    copy gmail pass

📥 Import:
    import 'path/to/file.txt'

Importing requires the below data format. Each line being a new record
'gmail' user = 'joseph ballin' sensitive pass = 'ни шагу назад, товарищи!'
'discord' user = 'pablo susscobar' pass = 'plata o plomo'

🔐 Change Master Password: chmpw
```

## 🌟 Connect with Us

M. Zahash – zahash.z@gmail.com

Distributed under the MIT license. See `LICENSE` for more information.

[https://github.com/zahash/](https://github.com/zahash/)

## 🤝 Contribute to RoyalGuard!

1. Fork it (<https://github.com/zahash/royalguard/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

## ❤️ Show Some Love!

If RoyalGuard makes your life easier, consider giving it a [🌟 on GitHub!](https://github.com/zahash/royalguard/stargazers)

Thank you for choosing RoyalGuard - Your Secure and Ergonomic Password Manager! 🛡️🚀
