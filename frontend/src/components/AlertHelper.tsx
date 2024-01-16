import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
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
  onSubmit: (data: T) => unknown;
}

export default function AlertHelper<T>(props: AlertHelperProps<T>) {
  const [data, setData] = useState<T>(props.initialData);
  return (
    <AlertDialog
      onOpenChange={(o) => {
        if (!o && props.onCancel != null) {
          props.onCancel();
        }
      }}
    >
      <AlertDialogTrigger asChild>{props.trigger}</AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{props.title}</AlertDialogTitle>
          <AlertDialogDescription>
            {props.content({ data, setData })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={() => {
              props.onSubmit(data);
            }}
          >
            {props.submitAction}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
