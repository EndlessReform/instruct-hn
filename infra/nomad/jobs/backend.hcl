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
        image = "ghcr.io/endlessreform/backend:latest"

        port_map {
          http = 80
        }
      }

      service {
        name = "backend-service"
        port = "http"
      }
    }
  }
}