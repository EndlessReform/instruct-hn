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
        static = 3000  # Host port; matches the port exposed by the Docker container
        to     = 3000  # Container port; matches the port exposed by the Docker container
      }
    }

    task "backend-task" {
      driver = "docker"

      config {
        image = "ghcr.io/endlessreform/backend:canary"
        
        ports = ["http"]  # This should align with the 'port' label above
      }

      service {
        provider = "nomad"
        name     = "backend-service"
        port     = "http"
      }

      env {
        HN_API_URL="${var.hn_api_url}"
        TRITON_SERVER_ADDR="${var.triton_server_addr}"
        DB_URL="${var.db_url}"
      }
    }
  }
}
