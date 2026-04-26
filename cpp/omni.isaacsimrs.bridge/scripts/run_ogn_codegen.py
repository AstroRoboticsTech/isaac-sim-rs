"""Kit-side OmniGraph node codegen driver. Reads args from env vars, writes <ClassName>Database.h."""

import os
import omni.graph.tools.ogn as ogn
import omni.kit.app

ogn_file = os.environ["OGN_FILE"]
class_name = os.environ["OGN_CLASS_NAME"]
extension = os.environ["OGN_EXTENSION"]
module = os.environ.get("OGN_MODULE", extension)
out_dir = os.environ["OGN_OUT_DIR"]

with open(ogn_file) as f:
    raw = f.read()

result = ogn.code_generation(raw, class_name, extension, module)

os.makedirs(out_dir, exist_ok=True)
db = result.get("cpp")
if not db:
    raise RuntimeError(f"codegen produced no C++ database for {class_name}")

out_path = os.path.join(out_dir, f"{class_name}Database.h")
with open(out_path, "w") as f:
    f.write(db)

print(f"OGN_CODEGEN: wrote {out_path}", flush=True)

omni.kit.app.get_app().post_quit()
