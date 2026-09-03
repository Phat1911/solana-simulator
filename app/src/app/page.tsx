import { Dashboard } from "@/components/dashboard";
import { loadDeploymentConfig } from "@/lib/deployment";

export default function Page() {
  return <Dashboard initialDeployment={loadDeploymentConfig()} />;
}
