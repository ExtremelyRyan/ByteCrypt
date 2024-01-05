<p align="center">
<img src=images/ByteCrypt-hr.png align="center"  />
<h1 align="center">ByteCrypt</h1>
</p>
<br/>
 
Encryption command line application on single file or entire directories.

<h2 align="center"><FONT COLOR="RED">CAUTION </FONT></h2>
<h3>this is very much a <b>work in progress</b>, and is undergoing rapid development that may break between commits.</h3>

## What is ByteCrypt?

ByteCrypt came about because I wanted the conveience of using all of the popular cloud file storage options, without the worry of storing my documents that contained PII (personally identifiable information). 

I also wanted a way for people who are not adept in encryption, computers, etc. to be able to get a simple program that they can use as a additional layer of security to protect their most senitive files, while maintaining ease of use.

### Encryption
ByteCrypt uses [chacha20poly1305](https://en.wikipedia.org/wiki/ChaCha20-Poly1305) ([RFC 8439](https://datatracker.ietf.org/doc/html/rfc8439)) for it's encryption, using 256-bit keys and a unique 96-bit nonce for every file.

### Compression
file size can quickly get out of hand, especially when you are backing up to the cloud. That's why we use [Zstandard](https://en.wikipedia.org/wiki/Zstd) to compress files before encryption.

You can configure your level of compression from the configuration file.

### Storage
by default, we store a backup `.crypt` when encrypting, so you can be sure to always have a local copy.


## Getting Started

### Dependencies

currently being tested on windows 10,11, wsl(ubuntu), and Arch linux.
requires minimum Rust version > 1.70.

### Installing

* Clone the repository, and go to the root project directory. Run `cargo install --path .`


### Executing program

By default, we will create a new file (file<b>.crypt</b>) with the encrypted contents. this can be overidden with the `in_place` flag.
```bash 
crypt encrypt file.ext
```

Encrypt a file in-place
```bash
crypt encrypt -p file.ext
```

Encrypt a whole directory with a path! even include hidden files with `-i`
```bash
crypt encrypt /some/dir
crypt encrypt -i /some/dir
```

Decryption is just as easy for a file
```bash
crypt decrypt file.crypt
```
Decryption is just as easy
```bash
crypt decrypt file.crypt
```
if the original file is still there, no problem. we will rename the decrypted contents to <b>`file-decrypted.ext` </b> so you can easily tell the difference.

Don't want the extra crypt files, or would prefer to encrypt files in place? no problem, you can adjust that with the configuration command:

```bash
crypt config -u retain false
```

 
## Authors 
Creator Ryan M - Twitter [@Extremely_Ryan](https://twitter.com/Extremely_Ryan)


Josh B: ?? 


email: <thebytecrypt@gmail.com>

## Version History

* 0.1 WIP
    * Initial Release

## Current issues:

1. Cloud Upload & Download
    * Currently still in the testing phase, and will not be available to the public until cloud submission to google.    
2. missing status messages when doing encryption / decryption. 




## License

This project is licensed under the [MIT] License - see the LICENSE.md file for details

## Acknowledgments

Inspiration, code snippets, etc.
* [awesome-readme](https://github.com/matiassingers/awesome-readme) 
