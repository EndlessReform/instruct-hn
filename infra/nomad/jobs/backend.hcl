job "backend" {
  datacenters = ["dc1"]

  group "backend-group" {
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