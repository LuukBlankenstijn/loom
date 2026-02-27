# Loom

This mono-repo contains all components that make up the Loom system. The Loom system is a system that automates and simplifies tasks surrounding contest machines in Algorithmic programming contest in the ICPC style.

## Features

- Wallpaper distribution
- Integration with ICPC spec compliant contest system
  - gets contest and team information from the contest system
  - displays a countdown right before the contest starts and unlocks it after countdown
  - sets the ip of a team in the contest system for auto login
- Drawing a map of the contest area

### Features planned

- assigning teams to a location on the map
- running custom commands on the team machine via the web-ui
- lock and unlock team machines from the web ui

## Components

The system exist out of several components:

- Backend: go connectRPC server, central brain of the system. Has stream connections to the different clients and has some in-memory state about contest machines
- Greeter: Greetd greeter running on the contest machine
- Station: systemd service running on team machine written in rust. This acts as the bridge between the backend and the greeter. This is done to make sure the greeter still functions when the rest of the system breaks down.
- Dashboard: This is the web ui that is used to manage everything. It is written in typescript with a generated connectrpc client
- map-editor: Map editor iced component. This is compiled to web assembly and embedded into the dashboard. It is also going to be used on the client machine to allow assigning teams to a location on the machine itself.
