# About
Recursive links extraction from a site without repetition(repetition of a link is determined by a [link unifier](src/link_unificator.rs)).
# Examples
* As an example, you can run test from [main file](src/main.rs#L64-L79), it will extract all links that can be reached 
from those links that specified in [the test file](example/init_links.txt) and save it in `./example/result.txt`.
* You can run the same example from console: `cargo run -- -s "./example/result.txt" links-file "./example/init_links.txt"`.
* Or you can list all initial links: `cargo run -- -s "./example/result.txt" links "%link_1%" ... "%link_N%"`.
# CLI parameters:
```
Usage: site_link_extract.exe [OPTIONS] <COMMAND>

Commands:
  links-file  load initial links from specified file
  links       specify initial links in console
  help        Print this message or the help of the given subcommand(s)

Options:
  -s <SAVE_TO>        file to save extracted links (if not specified then they will be printed in console)
  -m <MAX_ASYNC>      max number of async page loads at the same time (by default: inf)
  -d                  disable printing `+` after each loaded page
  -h, --help          Print help
```
