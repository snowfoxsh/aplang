# Preamble
Hello contributor. First, thank you for your possible contribution. Big or small, I welcome all contributions from all skill levels. Adding and working on the book is a great place to start as the interpreter can sometimes be intimidating.

Lets get you onboarded!

# Editing the book
You will need `mdbook` installed. Go ahead and install it 

```shell
cargo install mdbook
```

You can host the book server with 

```shell
mdbook serve
```

## Editors
You can use any markdown editor you would like, its just markdown after all. Some good options are Obsidan.md and VSCode with the appropriate extensions. Both are good. Do *not* commit editor specific configs. 

## More
- Read about mdbook
	- https://rust-lang.github.io/mdBook/
- Obsidan
	- https://obsidian.md/


## Continuous integration
The book is automatically deployed via GitHub Actions upon pull or commit. 

The action does not cache shit so its a bit slow but oh well (i don't really care). If u want to ostomies the build time go ahead. 


# AI Slop
Do not commit ChatGPT AI Slop or you will be gently drop-kicked. This world has enough slop. **Using AI with care is good and OK.** 

If your edit includes "delve" or "lets delve into it!" it will be thrown out `/j`. 
# Writing style
Be *clear*, be *explicit*. You are writing for the ears of absolute beginners. Beginners have what some would consider "silly questions", they are *far from* silly. Be explicit!

Things are hard... Be clear and provide directions. You often times cannot provide to much detail, this is technical writing, you are not writing a new your time best selling novel. 

The guide is written in the style of ["Rust By Example"](https://doc.rust-lang.org/rust-by-example/) and ["The Rust Book"](https://doc.rust-lang.org/book/).

# Structure of the book

Right now there is just one "book". It includes all the user facing documentation and information. If this project gains traction and community support the time might come when the standard lib documentation will be separated from the guide.

## Guide
The first section is a "guide". It should teach the user how to use AP Lang. There is pretty much no other documentation for the APCSP Pseudocode aside from the reference sheet and some practice tests. Its about time that something is done about that (being as this is a popular AP Class that hundreds of thousands of students take each year, like what the freak?). 

Rust by example is great but it assumes the reader is fluent in C++, which makes sense. We don't have that luxury. 

## STD
The second section provides documentation for the standard library. Whenever the main repository is changed this must be updated! 

# Misc
Make sure your code works as intended (obviously). If you are intending to contribute to the codebase itself please see the contributing guide in the respective repo. This guide is for contributing to the book.

# In Progress
- [ ] Better Errors:
	- [ ] Write better error messages
 	- [ ] Interpreter report object is clunky 

