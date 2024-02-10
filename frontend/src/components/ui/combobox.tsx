/* eslint-disable @typescript-eslint/strict-boolean-expressions */
import * as React from "react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { RxCheck, RxChevronUp } from "react-icons/rx";

interface ComboboxProps {
  options: { value: string; label: string }[];
  onChange: (value: string) => unknown;
  name: string;
  value: string;
  disabled?: boolean | undefined;
  title?: string | undefined;
}
export function Combobox({
  options,
  onChange,
  name,
  value,
  disabled,
  title,
}: ComboboxProps) {
  const [open, setOpen] = React.useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild disabled={disabled}>
        <Button
          title={title}
          disabled={disabled}
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-[200px] justify-between"
        >
          {value
            ? options.find((o) => o.value === value)?.label
            : `Select ${name}...`}
          <RxChevronUp className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command>
          <CommandInput placeholder="Search..." />
          <CommandEmpty>No option found.</CommandEmpty>
          <CommandGroup>
            {options.map((o) => (
              <CommandItem
                key={o.value}
                value={o.value.toLowerCase()}
                onSelect={(currentValue) => {
                  onChange(o.value === value ? "" : o.value);
                  setOpen(false);
                }}
              >
                <RxCheck
                  className={cn(
                    "mr-2 h-4 w-4",
                    value === o.value ? "opacity-100" : "opacity-0",
                  )}
                />
                {o.label}
              </CommandItem>
            ))}
          </CommandGroup>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
