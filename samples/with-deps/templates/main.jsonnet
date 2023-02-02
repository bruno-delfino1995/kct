// Copyright 2018 grafana, sh0rez
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

local _ = import "kct.libsonnet";
local k = import "github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet";

{
	// use locals to extract the parts we need
	local deploy = k.apps.v1.deployment,
	local container = k.core.v1.container,
	local port = k.core.v1.containerPort,
	local service = k.core.v1.service,

	// defining the objects:
	grafana: {
		// deployment constructor: name, replicas, containers
		deployment: deploy.new(name=_.input.name, replicas=1, containers=[
			// container constructor
			container.new(_.input.name, "grafana/grafana")
			+ container.withPorts( // add ports to the container
					[port.new("ui", _.input.port)] // port constructor
				),
		]),

		// instead of using a service constructor, our wrapper provides
		// a handy helper to automatically generate a service for a Deployment
		service: k.util.serviceFor(self.deployment)
						 + service.mixin.spec.withType("NodePort"),
	}
}
