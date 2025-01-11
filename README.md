<div align="center">
<h3></h3> <!-- gap between the title and the readme div, there's prob a better way but eh -->
<img width="512" src="assets/branding/Quark_Full.svg">

<i>"Quarks are very small particles..."</i>

<a href="https://codeberg.org/pparaxan/Quark/releases" target="_blank"><img src="https://img.shields.io/gitea/v/release/pparaxan/Quark?gitea_url=https%3A%2F%2Fcodeberg.org%2F&include_prereleases&sort=semver&display_name=release&date_order_by=published_at&style=for-the-badge&logo=codeberg&logoColor=white&color=%232185D0"/></a>
<a href="https://crates.io/crates/libquark" target="_blank"><img src="https://img.shields.io/crates/size/libquark?style=for-the-badge&logo=rust&logoColor=white&color=%23ffc933"/></a>
<a href="https://discord.gg/S6cfRda2DU" target="_blank"><img src="https://img.shields.io/badge/discord%20server-5865F2?style=for-the-badge&logo=discord&logoColor=white"/></a>
<a href="https://pparaxan.codeberg.page/Quark" target="_blank"><img src="https://img.shields.io/badge/website-ffb4b4?style=for-the-badge&logo=Codeberg&logoColor=white"/></a>
</div>

# Introduction
Quark is a "do-it-yourself" framework designed to create fast and tiny applications for all major desktop platforms by using your system's webview.

Developers can build their user interface using HTML, CSS, and JavaScript, paired with a high-performance Rust backend for speed and reliability.

# Features
* `localhost` free
* Built-in application bundler (via the `bundle` feature) to create app installer in formats like
    * deb
    * exe

And more soon to come!

# Platforms

Quark supports distribution and development on the following platforms:

| Platform          | Versions       |
| ----------------- | -------------- |
| Linux             | webkit2gtk 4.1 |
| macOS             | -              | <!-- I don't have a macbook, heck I don't even know if Quark works on it :skull: -->
| Windows           | 10+            |