variable "REGISTRY" {
  default = "reg.serabass.kz/vibecoding/branchy"
}

variable "TAG" {
  default = "latest"
}

group "default" {
  targets = ["gateway", "backend"]
}

target "gateway" {
  context    = "."
  dockerfile = "Dockerfile.gateway"
  tags       = ["${REGISTRY}/gateway:${TAG}"]
  platforms  = ["linux/amd64"]
  cache-from = ["type=registry,ref=${REGISTRY}/gateway:buildcache"]
  cache-to   = ["type=registry,mode=max,ref=${REGISTRY}/gateway:buildcache"]
}

target "backend" {
  context    = "."
  dockerfile = "Dockerfile.backend"
  tags       = ["${REGISTRY}/backend:${TAG}"]
  platforms  = ["linux/amd64"]
  cache-from = ["type=registry,ref=${REGISTRY}/backend:buildcache"]
  cache-to   = ["type=registry,mode=max,ref=${REGISTRY}/backend:buildcache"]
}
