import { ResourceInternalHealth } from "si-entity";
import { OpSource } from "si-entity/dist/siEntity";
import { Qualification } from "si-registry";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import { awsAccessKeysEnvironment, awsKubeConfigPath } from "../support";
import WebSocket from "ws";
//import Debug from "debug";
//const _debug = Debug("veritech:controllers:intel:dockerImage");

function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const entity = request.entity;

  entity.set({
    source: OpSource.Inferred,
    system: "baseline",
    path: ["name"],
    value: request.entity.name,
  });

  return { entity: request.entity };
}

export async function syncResource(
  ctx: typeof SiCtx,
  req: SyncResourceRequest,
  _ws: WebSocket,
): Promise<CommandProtocolFinish["finish"]> {
  const system = req.system.id;
  const defaultArgs = ["get", "--raw=/readyz?verbose"];
  const response: CommandProtocolFinish["finish"] = {
    data: {},
    state: req.resource.state,
    health: req.resource.health,
    internalStatus: req.resource.internalStatus,
    internalHealth: req.resource.internalHealth,
    subResources: req.resource.subResources,
  };
  let awsEnv;
  try {
    awsEnv = awsAccessKeysEnvironment(req);
  } catch (e) {
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    response.state = "error";
    response.error = `Cannot find AWS access keys! (cause=${e})`;
    return response;
  }
  const kubeConfigDir = await awsKubeConfigPath(
    req,
    req.entity.getProperty({ system, path: ["name"] }),
  );
  const result = await ctx.exec(
    "kubectl",
    [...defaultArgs, "--kubeconfig", `${kubeConfigDir.path}/config`],
    { env: awsEnv, reject: false },
  );
  if (result.exitCode != 0) {
    response.error = result.all;
    response.state = "not ready";
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
  } else {
    response.data["readyz"] = result.all.split("\n");
    response.state = "ready";
    response.health = "ok";
    response.internalHealth = ResourceInternalHealth.Ok;
  }
  const nodesResult = await ctx.exec(
    "kubectl",
    [
      "get",
      "nodes",
      "-o",
      "json",
      "--kubeconfig",
      `${kubeConfigDir.path}/config`,
    ],
    { env: awsEnv, reject: false },
  );
  if (nodesResult.exitCode != 0) {
    response.error = nodesResult.all;
    response.state = "nodes failing";
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
  } else {
    response.data["nodes"] = JSON.parse(nodesResult.stdout);
  }

  return response;
}

export default { inferProperties, syncResource };
