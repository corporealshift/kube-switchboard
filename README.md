# Kube Switchboard
A graphical switchboard for viewing/interacting with kubernetes resources. Kind of like K9s, but worse.

## Config
This tool is (somewhat) configurable. There is a `Config.toml` file included which you can modify
to do what you want. Below are the options for your Config file:

### [kubernetes.expected]
You can enable expected services and deployments in the status board by setting these two configuration values:
- `services`: An array of service names to check for.
- `deployments`: An array of deployment names to check for.

### [switchboard]
On the main screen you can add links and actions.
- `links`: An array of urls to provide links to. Currently will unwrap `{namespace}` into the namespace you have selected.
  Example: `{url = "https://{namespace}.mysite.com", name="Mysite"}` will provide a link to `https://hello.mysite.com` when you pick the `hello` namespace.
- `actions`: Enables buttons that run kubernetes commands and prints the result. Currently only works for `get-secret`.i
  Example: `{action = "get-secret", resource="my-kube-secret", name="My Secret"}`
-- `action`: The action to run. Can only be set to `get-secret` at the moment.
-- `resource`: The name of the resource to act on.
-- `name`: The name to use for this action.

## Dev
Uses egui for the ui, tokio for the async runtime environment, and kube for interacting with kubernetes.