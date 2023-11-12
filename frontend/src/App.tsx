import { createContext, useEffect, useState } from "react";
import "./App.css";
import { Button } from "./components/ui/button";
import { useNavigate, useLocation, Outlet } from "react-router-dom";
import { type OCELInfo } from "./types/ocel";
import MenuLink from "./components/MenuLink";
import Spinner from "./components/Spinner";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import toast from "react-hot-toast";

export const OcelInfoContext = createContext<OCELInfo | undefined>(undefined);

function App() {
  const [loading, setLoading] = useState(false);
  const [ocelInfo, setOcelInfo] = useState<OCELInfo>();

  const navigate = useNavigate();
  const location = useLocation();
  const isAtRoot = location.pathname === "/";

  const [availableOcels, setAvailableOcels] = useState<string[]>([]);
  const [selectedOcel, setSelectedOcel] = useState<string>();
  useEffect(() => {
    fetch("http://localhost:3000/ocel/info", { method: "get" })
      .then(async (res) => {
        if (res.ok) {
          const json: OCELInfo = await res.json();
          console.log({ json });
          setOcelInfo(json);
        }
      })
      .catch((e) => {
        // console.error(e);
      });
    fetch("http://localhost:3000/ocel/available", { method: "get" })
      .then(async (res) => {
        const json: string[] = await res.json();
        setAvailableOcels(json);
        console.log(json);
      })
      .catch((e) => {
        console.error(e);
      });
  }, []);

  async function loadOcel() {
    const res = await fetch("http://localhost:3000/ocel/load", {
      method: "post",
      body: JSON.stringify({ name: selectedOcel }),
      headers: { "Content-Type": "application/json" },
    });
    const ocelInfo: OCELInfo = await res.json();
    setOcelInfo(ocelInfo);
  }

  return (
    <OcelInfoContext.Provider value={ocelInfo}>
      <div className="max-w-full overflow-hidden h-screen text-center grid grid-cols-[15rem_auto]">
        <div className="bg-gray-50 border-r border-r-slate-200 px-2">
          <div className="flex justify-center py-2 px-2">
            <h1 className="text-pink-500 opacity-90 py-1 text-2xl font-black">
              OCED
            </h1>
            <h1 className="text-blue-500 opacity-90 py-1 text-2xl font-black -ml-[0.5ch]">
              DECLARE
            </h1>
          </div>
          <div className="flex flex-col gap-2">
            {ocelInfo !== undefined && (
              <span className="flex flex-col items-start mx-auto">
                <span className="font-mono">{ocelInfo.num_events} Events</span>
              </span>
            )}
            {ocelInfo !== undefined && (
              <>
                <MenuLink to="/ocel-info">View OCEL Info</MenuLink>
                <MenuLink to="/beta">Open Beta</MenuLink>
              </>
            )}
            <br />
            {!isAtRoot && (
              <>
                <MenuLink
                  to={"/"}
                  onClick={(ev) => {
                    ev.preventDefault();
                    navigate(-1);
                  }}
                >
                  Back
                </MenuLink>
              </>
            )}
          </div>
        </div>
        <div className="px-4 overflow-auto">
          <div className="flex justify-center px-2 pt-4">
            <h1 className="text-pink-500 opacity-90 py-1 text-5xl font-black">
              OCED
            </h1>
            <h1 className="text-blue-500 opacity-90 py-1 text-5xl font-black -ml-[0.5ch]">
              DECLARE
            </h1>
          </div>

          {/* <Spinner loadingText="Importing OCEL..." spinning={loading} /> */}
          {isAtRoot && (
            <div className="">
              <Select
                value={selectedOcel}
                onValueChange={(v) => {
                  setSelectedOcel(v);
                }}
              >
                <SelectTrigger className={"w-[180px] mx-auto my-2"}>
                  <SelectValue placeholder="Select an OCEL" />
                </SelectTrigger>
                <SelectContent>
                  {availableOcels.map((ocelName) => (
                    <SelectItem key={ocelName} value={ocelName}>
                      {ocelName}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                disabled={loading || selectedOcel === undefined}
                size="default"
                onClick={async () => {
                  setLoading(true);
                  await toast
                    .promise(loadOcel(), {
                      loading: "Loading OCEL...",
                      success: "Loaded OCEL",
                      error: "Failed to load OCEL",
                    })
                    .finally(() => {
                      setLoading(false);
                    });
                }}
              >
                {loading && <Spinner />}
                <span>Open JSON OCEL</span>
              </Button>
            </div>
          )}
          <div className="w-full h-full">
            <Outlet />
          </div>
        </div>
      </div>
    </OcelInfoContext.Provider>
  );
}

export default App;
