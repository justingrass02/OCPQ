import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { createContext, useContext, useEffect, useState } from "react";
import toast from "react-hot-toast";
// import { Outlet, useLocation } from "react-router-dom";
import { PiGraph, PiSealCheckBold } from "react-icons/pi";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import "./App.css";
import { BackendProviderContext } from "./BackendProviderContext";
import MenuLink from "./components/MenuLink";
import Spinner from "./components/Spinner";
import { Button } from "./components/ui/button";
import { type OCELInfo } from "./types/ocel";
const VALID_OCEL_MIME_TYPES = [
  "application/json",
  "text/json",
  "text/xml",
  "application/xml",
];
export const OcelInfoContext = createContext<OCELInfo | undefined>(undefined);

function App() {
  const [loading, setLoading] = useState(false);
  const [ocelInfo, setOcelInfo] = useState<OCELInfo>();

  const location = useLocation();
  const navigate = useNavigate();
  const isAtRoot = location.pathname === "/";
  const [availableOcels, setAvailableOcels] = useState<string[]>([]);
  const [selectedOcel, setSelectedOcel] = useState<string>();
  const backend = useContext(BackendProviderContext);
  useEffect(() => {
    console.log({ backend });
    void backend["ocel/info"]().then((info) => {
      if (info !== undefined) {
        setOcelInfo(info);
      } else {
        setOcelInfo(undefined);
      }
    });
    if (backend["ocel/available"] !== undefined) {
      void toast
        .promise(backend["ocel/available"](), {
          loading: "Loading available OCEL",
          success: "Got available OCEL",
          error: "Failed to load available OCEL",
        })
        .then((res) => {
          setAvailableOcels(res);
        });
    }
  }, []);

  async function loadOcel() {
    if (selectedOcel == null) {
      console.warn("No valid OCEL selected");
      return;
    }
    if (backend["ocel/load"] === undefined) {
      console.warn("ocel/load is not supported by this backend.");
      return;
    }
    await toast.promise(
      backend["ocel/load"](selectedOcel).then((ocelInfo) => {
        setOcelInfoAndNavigate(ocelInfo);
      }),
      {
        loading: "Importing OCEL...",
        success: "Imported OCEL",
        error: "Failed to import OCEL",
      },
    );
  }

  function handleFileUpload(file: File | null) {
    if (backend["ocel/upload"] === undefined) {
      console.warn("No ocel/upload available!");
      return;
    }
    if (file != null) {
      void toast
        .promise(backend["ocel/upload"](file), {
          loading: "Importing file...",
          success: "Imported OCEL",
          error: "Failed to import OCEL",
        })
        .then((ocelInfo) => {
          if (ocelInfo != null) {
            setOcelInfoAndNavigate(ocelInfo);
          } else {
            setOcelInfo(undefined);
          }
        });
    }
  }

  const showAvailableOcels =
    availableOcels.length > 0 && backend["ocel/available"] !== undefined;
  const filePickerAvailable = backend["ocel/picker"] !== undefined;

  function setOcelInfoAndNavigate(info: OCELInfo | undefined) {
    setOcelInfo(info);
    if (info !== null) {
      navigate("/ocel-info");
    }
  }

  return (
    <OcelInfoContext.Provider value={ocelInfo}>
      <div className="max-w-full overflow-hidden h-screen text-center grid grid-cols-[15rem_auto]">
        <div className="bg-gray-50 border-r border-r-slate-200 px-2">
          <img
            src="/favicon.png"
            className="w-[7rem] h-[7rem] mx-auto mt-4 mb-2"
          />
          <h2 className="font-bold text-3xl bg-clip-text text-transparent bg-gradient-to-r from-slate-800 to-sky-600 tracking-tighter">
            OCPQ
          </h2>
          <div className="flex flex-col gap-2 mt-4">
            {ocelInfo != null && (
              <span className="flex flex-col items-center mx-auto text-xl">
                <span className=" font-semibold text-green-700">
                  OCEL loaded
                </span>
                <span>{ocelInfo.num_events} Events</span>
                <span>{ocelInfo.num_objects} Objects</span>
              </span>
            )}
            {ocelInfo != null && (
              <>
                <MenuLink to="/ocel-info">OCEL Info</MenuLink>
                <MenuLink to="/graph">
                  <PiGraph className="text-lg mr-1 text-purple-700" />
                  Graph
                </MenuLink>
                <MenuLink
                  to="/constraints"
                  classNames={[
                    "bg-purple-200 hover:bg-purple-300 border-purple-300",
                  ]}
                >
                  <PiSealCheckBold className="text-lg mr-1 text-purple-700" />
                  Constraints
                </MenuLink>
              </>
            )}
            <br />
            {!isAtRoot && (
              <>
                <MenuLink to={"/"}>Back</MenuLink>
              </>
            )}
          </div>
        </div>
        <div className="px-4 overflow-y-auto overflow-x-hidden py-4">
          {isAtRoot && (
            <h2 className="text-3xl font-semibold mb-4">Load OCEL</h2>
          )}
          {isAtRoot &&
            filePickerAvailable &&
            backend["ocel/picker"] !== undefined && (
              <Button
                disabled={loading}
                onClick={() => {
                  setLoading(true);
                  void toast
                    .promise(backend["ocel/picker"]!(), {
                      loading: "Loading OCEL2...",
                      success: "Imported OCEL2",
                      error: "Failed to load OCEL2",
                    })
                    .then((ocelInfo) => {
                      setOcelInfoAndNavigate(ocelInfo);
                    })
                    .finally(() => setLoading(false));
                }}
              >
                {loading && <Spinner />}
                Select an OCEL 2.0 File...
              </Button>
            )}
          {isAtRoot &&
            showAvailableOcels &&
            backend["ocel/load"] !== undefined && (
              <div className="">
                <Select
                  name="Select available OCEL"
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
                  <span>Load Selected OCEL</span>
                </Button>
              </div>
            )}

          {isAtRoot && backend["ocel/upload"] !== undefined && (
            <div>
              {showAvailableOcels && <div className="w-full my-4">OR</div>}
              <div
                className="flex items-center justify-center w-full max-w-2xl mx-auto"
                onDragOver={(ev) => {
                  ev.preventDefault();
                  const items = ev.dataTransfer.items;
                  if (items.length > 0 && items[0].kind === "file") {
                    const fileMimeType = items[0].type;
                    if (!VALID_OCEL_MIME_TYPES.includes(fileMimeType)) {
                      const fileType =
                        fileMimeType.length === 0 ? "" : `(${fileMimeType})`;
                      toast(
                        `Files of type ${fileType} are not supported!\n\nIf you are sure that this is an valid OCEL2 file, please select it manually by clicking on the dropzone.`,
                        { id: "unsupported-file" },
                      );
                    }
                  }
                }}
                onDrop={(ev) => {
                  ev.preventDefault();
                  const files = ev.dataTransfer.items;
                  if (files.length > 0) {
                    const fileWrapper = files[0];
                    const file = fileWrapper.getAsFile();
                    if (VALID_OCEL_MIME_TYPES.includes(file?.type ?? "")) {
                      handleFileUpload(file);
                    } else {
                      const fileType =
                        file?.type == null ? "" : `(${file?.type})`;
                      toast(
                        `Files of this type ${fileType} are not supported!\n\nIf you are sure that this is an valid OCEL2 file, please select it manually by clicking on the dropzone.`,
                        { id: "unsupported-file" },
                      );
                    }
                  }
                }}
              >
                <label
                  htmlFor="dropzone-ocel-file"
                  className="flex flex-col items-center justify-center w-full h-64 border-2 border-gray-400 border-dashed rounded-lg cursor-pointer bg-blue-50/20 hover:bg-blue-100/30"
                >
                  <div className="flex flex-col items-center justify-center pt-5 pb-6">
                    <p className="mb-2 text-sm text-gray-500">
                      <span className="font-semibold">
                        Click to select an OCEL file
                      </span>{" "}
                      or drag a file here
                    </p>
                    <p className="text-xs text-gray-500">
                      Supported: OCEL2-JSON, OCEL2-XML
                    </p>
                  </div>
                  <input
                    onChange={(ev) => {
                      if (ev.currentTarget.files !== null) {
                        handleFileUpload(ev.currentTarget.files[0]);
                      }
                    }}
                    id="dropzone-ocel-file"
                    type="file"
                    className="hidden"
                    accept=".json, .xml"
                  />
                </label>
              </div>
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
