# md_to_pdf

### Brief
Simple utility to convert from Markdown format into pretty PDF document. 
It is written in Rust programming language using several libraries:
- `comrak` to convert markdown into HTML,
- `wkhtmltopdf` to parse HTML with embedded CSS and render PDF document.

### How it works
This small console app converts Markdown text directly into HTML document,
which is extended and decorated by CSS styles. Full CSS style is embedded
into the application as a resource and then inserted into the HTML document.
After that application parses some HTML tokens and extends them by adding
custom classes to them. This HTML document is converted into PDF file.

### Installing


#### Linux
Depends on [wkhtmltopdf][1] that can be downloaded and installed with a package 
manager as such as apt or pacman. After installing this dependency run:
```
cargo install --git https://github.com/GeorgyFirsov/md_to_pdf.git
```


### Usage
```shell
./md_to_pdf -i ../my_awesome_document.md -o ../output/my_awesome_document.pdf
```

### Example
Current Markdown document (this README.md) is converted into the following PDF:
![Example](./docs/images/example.png)

[1]: https://wkhtmltopdf.org/downloads.html
