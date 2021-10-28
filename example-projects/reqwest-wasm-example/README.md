## Example usage of CLoudEvents sdk/Reqwest from WASM

First, ensure you have [`wasm-pack` installed](https://rustwasm.github.io/wasm-pack/installer/)

Then install the dependencies:

    npm install

And finally run the example:

    npm run serve

You should see a form in your browser at http://localhost:8080. When
the form is submitted, a CloudEvent will be sent to the Target URL,
http://localhost:9000 by default, which is the default URL for the
[actix example](../actix-web-example). Fire it up in another terminal
to verify that the data is successfully sent and received.

Open the javascript console in the browser to see any helpful error
messages.

This example is loosely based off of [this
example](https://github.com/seanmonstar/reqwest/tree/master/examples/wasm_github_fetch).
