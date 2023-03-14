# Kattis checker
This tool queries kattis for problems that are stored locally and haven't been uploaded and vice-versa.

There are no guarantees here, i wrote this in 30 minutes.

## Run
```
cargo run -- --cookie <contents of the EduSiteCookie cookie> --problems-dir <where you store your kattis solutions>
```

There is some caching functionality built in so you can run
```
cargo run -- --cookie <contents of the EduSiteCookie cookie> --problems-dir <where you store your kattis solutions> -o <file>
```
To save the queried problems, to load and not query you run 
```
cargo run -- --cookie <contents of the EduSiteCookie cookie> --problems-dir <where you store your kattis solutions> -i <file>
```
