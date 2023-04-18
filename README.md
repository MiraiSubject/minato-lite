# Minato Lite

This is a Rust backend server designed to act as the OAuth 2 middleman for an application stack I'm working on which uses osu! for the SSO. The problem we ran into is that we have different methods for dealing with the user's data depending on the microservice being called. Creating multiple osu! OAuth 2 apps and maintaining IDs and Secrets is a big hassle, so instead I developed this backend that sits in between the application stack and the osu! OAuth 2 endpoints which passes through the `authorization_code` to the correct app in the stack.

We only care about getting the `authorization_code` from the osu! callback, which is exactly what's being forwarded back to the callback.

## How does this work?

Originally when you craft the request for the `authorization_code` flow, you craft the following URL: https://example.com/oauth/authorize?client_id={}&redirect_uri={}&response_type={}&scope={}&state={}

Then the applications sends the user towards it. The user completes the login and consent flow, and gets redirected to the specified `redirect_uri`. This will be the callback URL in your backend, which exchanges the `code` for an access token by going to https://example.com/token and can then do requests on your behalf. 

The flow in this situation is as follows: 

`https://example.com/oauth/authorize -> https://yourstack.tld/callback` 

Instead of going to the aforementioned `authorize` URL, the application will instead be configured to go to the following URL: 
https://thisbackend.tld/authorize?client_id={}&redirect_uri={}&source_service={}

`source_service` equals your original callback URL. The Callback URL for your osu! OAuth 2 app should then be modified to use `https://thisbackend.tld/callback` instead. What happens here is that your application follows the following flow instead:  

`https://thisbackend.tld/authorize -> https://example.com/oauth/authorize -> https://thisbackend.tld/callback -> https://yourstack.tld/callback`

Now of course if you just deploy a single web application you will not benefit from this application at all. However once you start scaling to multiple applications that live on different subdomains, let's say for example:

- https://signup.yourstack.tld/
- https://dashboard.yourstack.tld/
- https://admin.yourstack.tld/

All these 3 apps live separately from each other, and thats where this application comes in. They will use this central backend to get the `authorization_code` and can exchange it themselves for the access token and fetching data. 

(This explanation could use some improvement)

## Development/Running

### Requirements
- Rust/Crablang installed on your system.

Copy the `.env.example` to `.env` and fill your osu! client ID and redirect URL. Then in `allowed_origins` add the callback URLs you want to accept separated by commas. If the exact URL is not included then you will get an Unauthorized error. The following is an example for using multiple allowed origins: 

`allowed_origins=http://localhost:8080/echo,http://localhost:8000/auth/osu/callback,http://localhost:3000/auth/osu/callback`

Run `cargo run` to run the application on http://localhost:8080/.

## Deploy

TODO

## Security

To prevent just anyone from abusing this backend when it gets to production there's an `allowed_origins` environment variable that only allows those specified origins to actually access the real authorization flow. This is to mimic the `redirect_uri` property in OAuth 2. However, I ship this with absolutely no warranty. 