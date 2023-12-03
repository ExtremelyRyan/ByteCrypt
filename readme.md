<p align="center">
<img src=images/ByteCrypt.png align="center" />
<h1 align="center">ByteCrypt</h1>
</p>
<br/>
 
file / directory encryption with future to upload and download encrypted files to cloud storage.

<h2 align="center">CAUTION</h2>
this is very much a work in progress, and is undergoing rapid development that may break between commits.

## What is ByteCrypt?

ByteCrypt came about because I wanted the conveience of using all of the popular cloud file storage options, without the worry of storing my documents that contained PII (personally identifiable information). 

## Getting Started

### Dependencies

currently being tested on windows 10,11, wsl(ubuntu), and Arch linux.
requires min Rust 1.70.

### Installing

* Clone the repository, and go to the root project directory. Run `cargo install --path .`

will publish crate once MVP is finished.

### Executing program

By default, we will create a new file with the encrypted contents. 

* Encrypt a file with
```
crypt encrypt foo.txt
```
* encrypt a whole directory with a path!
```
crypt encrypt /some/dir
crypt encrypt -i /some/dir (include hidden files)

```

 
## Authors 

Ryan : Twitter [@Extremely_Ryan](https://twitter.com/Extremely_Ryan)
Josh : 

## Version History

* 0.1 WIP
    * Initial Release

## License

This project is licensed under the [MIT] License - see the LICENSE.md file for details

## Acknowledgments

Inspiration, code snippets, etc.
* [awesome-readme](https://github.com/matiassingers/awesome-readme) 
