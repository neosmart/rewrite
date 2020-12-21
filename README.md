# A `rewrite` primer

**What it is**

`rewrite` is a simple command-line utility that allows for the in-place rewrite of a file's contents, even where the file is being read from as the input. This makes transforming the contents of a file via other standard unix utilities dead simple, even when they expect the input and output files/streams to be physically separate.

**The problem**

You have a sequence of chained operations/commands that reads from a given file `f`, and you want to replace the contents of `f` with the result of that chain of commands. If you try to redirect the output of your script via something like `> f` or `| tee f`, you'll find that more often than not, you'll lose everything and corrupt your datastream. That's because the upstream command is reading from the same file that is being written to, overwriting the input with the output.

**The solution**

`rewrite` makes it stupid easy to work around this problem. Just pipe the output of your workflow to `rewrite f` and you'll get the result you expected. Easy peasey!

**An example**

Say we want to sort a file. We don't want to sort _a copy_ of the file, we want to sort _the file itself_ (obviously). Unfortunately, it's not that easy. Here's an example wherein we select 1024 random words from a dictionary file and then want to sort the output. 

```
shuf -n 1024 /usr/share/dict/words  > words.txt
```

We can easily sort this list with the `sort` utility, but what happens when we try to save the output to itself?

```
sort words.txt > words.txt #don't do this!
```

This will result in a complete loss of data, as the shell will set up the output file handle before `sort` gets a chance to open the same file to read it. In the end, you get neither this nor that and lose all data in the process! 

Here's what you would normally do instead:

```
sort words.txt > temp
mv temp words.txt
```

Which is easy & straightforward enough, except when `sort` is part of a bigger workflow or a script, or when you forget, or when `temp` already exists, or when you don't have as straightforward of a case and don't realize that your source and destination files are one and the same. `rewrite` to the rescue!

Here's how simple using `rewrite` here would be:

`sort words.txt | rewrite words.txt`

Internally, `rewrite` does all the "magic" of reading from `stdin` and buffering the content until the upstream command has finished executing, then writing the output to the named file accordingly.



# Installing `rewrite`

`rewrite` is written in rust for performance, safety, and out-of-the-box cross-platform support. Installing `rewrite` (presuming it's not already available as a binary for your platform in your favorite package manager) is as simple as 

```
cargo install rewrite
```

Pre-built binaries for Windows and OS X are available separately.