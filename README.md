# EnvDNS - Environment-Driven DNS for Developers

![status: work in progress](https://img.shields.io/badge/status-WIP-yellow)

EnvDNS is an innovative tool designed to offer dynamic DNS management by borrowing the concept of environment-driven configurations, similar to how [direnv](https://direnv.net/) manages environment variables.

With EnvDNS, local development is taken a step further, providing developers the ability to switch between different DNS configurations with ease and efficiency.

## Features

- **Dynamic DNS Loading**: Automatically change DNS settings as you move between different projects.
- **Simple Configuration**: Just drop a `.envdnsrc` file in your project root and define your DNS preferences.
- **Integration Friendly**: Plays nicely with most modern development environments and tools.

## Installation

**Note**: The project is currently in development. Installation details will be provided once we have our initial release.

## Usage

1. Navigate to your project directory.
2. Create a `.envdnsrc` file.
3. Define your DNS preferences.
4. Let EnvDNS manage the DNS settings for you dynamically.

Example `.envdnsrc`:

```ini
# Sample configuration format
192.168.23.120  my-app-2.local.dev
192.168.23.200  db1.local.dev
```

## Contributing

Community contributions are highly encouraged. If you're eager to enhance EnvDNS or tackle issues, our [CONTRIBUTING.md](./CONTRIBUTING.md) guide is a great place to start.

## License

EnvDNS is licensed under the MIT License. Refer to the [LICENSE](./LICENSE) file for comprehensive details.

## Acknowledgements

Inspiration drawn from [direnv](https://direnv.net/).
