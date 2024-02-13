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

interface AlertHelperProps<T> {
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

export default function AlertHelper<T>(props: AlertHelperProps<T>) {
  const [data, setData] = useState<T>(props.initialData);
  return (
    <AlertDialog
      onOpenChange={(o) => {
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
            onClick={(ev) => {
              props.onSubmit(data, ev);
            }}
          >
            {props.submitAction}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
