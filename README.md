# CHAIRA 
chaira is an "airtable alternative", it lets u save data in a spreadsheet, for now the ui is really basic, thats bc i focused a lot on the backend (and will still do :pf:), and sadly the ui doesnt show all the functionalities that are present in the backend code, im still going to work on it, as i really want to replace airtable so hc can use it ;] 

### how to run 
this project requires, well, a lot of tools to comple, but because im smort and used nix you just need nix the package manager, and then run `nix develop`, this will install everything and even run redis and surrealdb for you, then u can just run `cargo leptos watch` inside and boom, it will run the thing and with hot reloads!
you can also use this as an input, and use it as a service via `services.chaira` 
