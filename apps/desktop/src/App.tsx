import { useState } from "react";

import { Layout, type Screen } from "./components/Layout";
import { DeviceDetailsScreen } from "./features/devices/DeviceDetailsScreen";
import { DevicesScreen } from "./features/devices/DevicesScreen";
import { DiagnosticsScreen } from "./features/diagnostics/DiagnosticsScreen";
import { DisplaysScreen } from "./features/displays/DisplaysScreen";
import { LightingScreen } from "./features/lighting/LightingScreen";
import { MacrosScreen } from "./features/macros/MacrosScreen";
import { ProfilesScreen } from "./features/profiles/ProfilesScreen";
import { DaemonProvider } from "./hooks/useDaemon";

function Screens() {
  const [screen, setScreen] = useState<Screen>("devices");
  const [selected, setSelected] = useState<string | null>(null);

  const openDevice = (interfaceId: string): void => {
    setSelected(interfaceId);
    setScreen("device");
  };
  const openLighting = (interfaceId: string): void => {
    setSelected(interfaceId);
    setScreen("lighting");
  };

  return (
    <Layout screen={screen} onNavigate={setScreen}>
      {screen === "devices" ? <DevicesScreen onOpenDevice={openDevice} /> : null}
      {screen === "device" ? (
        <DeviceDetailsScreen
          interfaceId={selected}
          onOpenLighting={openLighting}
          onBack={() => setScreen("devices")}
        />
      ) : null}
      {screen === "lighting" ? <LightingScreen interfaceId={selected} /> : null}
      {screen === "profiles" ? <ProfilesScreen /> : null}
      {screen === "macros" ? <MacrosScreen /> : null}
      {screen === "displays" ? <DisplaysScreen /> : null}
      {screen === "diagnostics" ? <DiagnosticsScreen /> : null}
    </Layout>
  );
}

export function App() {
  return (
    <DaemonProvider>
      <Screens />
    </DaemonProvider>
  );
}
