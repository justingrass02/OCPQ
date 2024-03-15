import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { useState } from "react";
import Spinner from "./Spinner";

type AlertHelperProps<T> =
  | {
      mode?: "normal";
      trigger: React.ReactNode;
      title: React.ReactNode;
      initialData: T;
      content: React.FC<{ data: T; setData: (data: T) => unknown }>;
      submitAction: React.ReactNode;
      onCancel?: () => unknown;
      onSubmit: (
        data: T,
        ev: React.MouseEvent<HTMLButtonElement, MouseEvent>,
      ) => unknown;
    }
  | {
      mode: "promise";
      trigger: React.ReactNode;
      title: React.ReactNode;
      initialData: T;
      content: React.FC<{ data: T; setData: (data: T) => unknown }>;
      submitAction: React.ReactNode;
      onCancel?: () => unknown;
      onSubmit: (
        data: T,
        ev: React.MouseEvent<HTMLButtonElement, MouseEvent>,
      ) => Promise<unknown>;
    };

export default function AlertHelper<T>(props: AlertHelperProps<T>) {
  const [data, setData] = useState<T>(props.initialData);
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  return (
    <AlertDialog
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (!o && props.onCancel != null) {
          props.onCancel();
        }
        setData(props.initialData);
      }}
    >
      <AlertDialogTrigger asChild>{props.trigger}</AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{props.title}</AlertDialogTitle>
          {/* <AlertDialogDescription> */}
          <div className="text-sm text-gray-700">
            {props.content({ data, setData })}
          </div>
          {/* </AlertDialogDescription> */}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            disabled={loading}
            onClick={(ev) => {
              if (props.mode === "promise") {
                ev.preventDefault();
                setLoading(true);
                void props.onSubmit(data, ev).finally(() => {
                  setLoading(false);
                  setOpen(false);
                });
              } else {
                props.onSubmit(data, ev);
              }
            }}
          >
            {loading && <Spinner />}
            {props.submitAction}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
