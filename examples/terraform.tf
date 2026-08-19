terraform {
  required_providers {
    containeros = {
      source = "frederic-arr/containeros"
    }
  }
}

provider "containeros" {}

resource "containeros_config_push" "qemu" {
    server = "127.0.0.1:50000"
    config = file("./noinstall.yaml")
}
