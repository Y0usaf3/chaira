# CHAIRA 
chaira is an "airtable alternative", it lets u save data in a spreadsheet, for now the ui is really basic, thats bc i focused a lot on the backend (and will still do :pf:), and sadly the ui doesnt show all the functionalities that are present in the backend code, im still going to work on it, as i really want to replace airtable so hc can use it ;] 

### how to run 
this project requires, well, a lot of tools to comple, first u need 
- [rust](https://rust-lang.org/tools/install/) 
- cargo-leptos `cargo install cargo-leptos` 
- [nodejs](https://nodejs.org/en/download)
- [bun](https://bun.com/)
- and a really powerful cpu bc it will take a while to compile ;-;
- [surrealdb](https://surrealdb.com/docs/self-hosted/installation)
  
to run the database, we first need to compile the db functions to a surli file that should be in the bit/ folder, for that, go to the bit/ folder and then build it by running 
```bash
surreal module build
``` 
a surli file will appear, now go to the root directory, and install the js libraries 
```bash
bun install .
```
now everything is set, we can finally compile the project to test it
```bash
cargo leptos serve
```
when changing the code, i recomend using `cargo leptos watch` which enables hot reloads 
