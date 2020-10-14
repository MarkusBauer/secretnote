SecretNote
==========

SecretNote is a website where users can store short, secret messages. 
These messages are end-to-end encrypted and can be read only once. 
In addition, users can also have end-to-end encrypted chats.

SecretNote is free and open source software, powered by Rust, Redis, Angular and Typescript.
It was mainly developed to learn some Rust basics.

Website: [https://secretnote.mk-bauer.de](https://secretnote.mk-bauer.de)


Host your own instance
----------------------
`TODO`


For developers
--------------
Building the secretnote server is a two-step process. You need Rust and a recent version of NodeJS.
Check out the build scripts for details.

**Installing dependencies:** `cargo fetch  && cd secretnote-fe &&  npm install`

**Building the Rust server:**
`cargo build --release`

**Building the frontend:**
`npm run build`
