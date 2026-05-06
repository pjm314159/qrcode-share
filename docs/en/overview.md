# Qrcode Share

## Background
Qrcode Share is a tool to share qrcode.  
First, someone need to scan the qrcode.  
Then, the qrcode will  be shared to the channel by server.  
Finally, the channel members can click the link and jump to the website.  
It's goal is to help sign the sign-in code

## Features
* channels support.
* Share qrcode to channel.
* Click the link and jump to the website.

## workflow
the app will run in web browser or wechat built-in browser.

### producer
user(don't need to login) -> create channel or input channel id ->join the channel -> pre-text the message-> scan qrcode and get the link -> share message to channel without confirm
### consumer
user(don't need to login) -> input channel id -> join the channel -> click the card -> jump to the website
if user chose jump to the link without confirm, it will jump to the website directly.(we will alert the user is unsafe.)
### message format
the messages in the channel will be like this:
```
[name]
[Qrcode link]
[expire time]
[type](optional)
[location](optional)
[time](auto generated)
```
the message will display in the form of card.  
you can see the card in the channel with the template.
```
[name]
[expire time](countdown)
[type](optional)
[qrcode link domain]
```
### channel format
the channel id is unique.
the channel's type like this :
```
[name]
[link limitation](optional)  -- to limit the qr_code link domian to insure the security.
[type](optional)
[location](optional)
[time](optional)
[teacher](optional)
```
the message in the channel is short-life.  
the message will be deleted after 1 hour.

the channel will be deleted if the channel noone send message in 30 days.
## tech stack
### backend
#### core framework
* rust
* axum - web framework with built-in WebSocket support
* tokio - async runtime

#### database & storage
* sqlx - compile-time checked SQL queries
* postgresql - primary database for channel metadata
* in-memory storage - for short-lived messages (1 hour TTL)

#### real-time communication
* axum WebSocket - real-time message push to channel members
* tokio::sync::broadcast - channel message broadcasting

#### serialization & data
* serde, serde_json - JSON serialization/deserialization
* uuid - unique identifier generation for channels and messages
* chrono - timestamp handling

#### logging & error handling
* tracing, tracing-subscriber - structured logging
* thiserror, anyhow - error handling

#### security & middleware
* tower-http - CORS, compression, and other middleware
* tower - service abstraction layer
* jsonwebtoken - optional channel access control (password protection)

#### configuration
* dotenvy - environment variable management

### frontend
#### core framework
* react 18+ - UI library
* typescript - type safety
* vite - build tool (faster than CRA)

#### state management
* zustand - lightweight state management
* react hooks - local component state

#### real-time communication
* native WebSocket API - connect to backend WebSocket server
* auto-reconnection logic

#### wechat integration
* weixin-js-sdk - WeChat JSSDK for scan QR code API
* @types/weixin-js-sdk - TypeScript type definitions

#### qr code processing
* jsQR or html5-qrcode - parse QR code from camera/image

#### routing & navigation
* react-router-dom - client-side routing

#### HTTP client
* axios or fetch - HTTP requests to backend API

#### styling
* tailwindcss - utility-first CSS framework
* postcss - CSS processing

#### utilities
* date-fns or dayjs - date/time manipulation
* clsx or classnames - conditional CSS classes

### development tools
#### testing
* cargo test - Rust unit testing
* vitest or jest - frontend unit testing
* @testing-library/react - React component testing

#### code quality
* eslint - JavaScript/TypeScript linting
* prettier - code formatting
* rustfmt - Rust code formatting
* clippy - Rust linter

#### API documentation
* utoipa or swagger - OpenAPI/Swagger documentation (optional)

#### version control
* git
* husky (optional) - git hooks for pre-commit checks

### deployment
#### containerization
* docker - containerization
* docker-compose - multi-container orchestration

#### web server
* nginx - reverse proxy, SSL termination, static file serving

#### SSL/HTTPS
* let's encrypt - free SSL certificates
* certbot - certificate management

#### monitoring (optional for small scale)
* basic logging aggregation
* health check endpoints
