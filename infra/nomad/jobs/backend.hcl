job "backend" {
  datacenters = ["dc1"]

  group "backend-group" {
    constraint {
      attribute = "${attr.cpu.arch}"
      value     = "amd64" # Constraining to x86 since backend likes CPU
    }

    network {
      mode = "bridge"

      port "http" {
        static = 8080
        to     = 80
      }
    }

    task "backend-task" {
      driver = "docker"

      config {
        # Moving to canary until i set up blue-green (if need be)
        image = "ghcr.io/endlessreform/backend:canary"

        port_map {
          http = 80
        }
      }

      service {
        provider = "nomad"
        name = "backend-service"
        port = "http"
      }
    }
  }
}