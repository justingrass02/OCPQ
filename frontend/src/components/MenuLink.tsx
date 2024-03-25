import { Link } from "react-router-dom";
import { buttonVariants } from "./ui/button";
import { type ReactNode } from "react";
import clsx, { type ClassValue } from "clsx";
type MenuLinkProps = {
  to: string;
  children: ReactNode;
  classNames?: ClassValue[];
  onClick?: (ev: React.MouseEvent<HTMLAnchorElement, MouseEvent>) => any;
};
export default function MenuLink(props: MenuLinkProps) {
  return (
    <Link
      className={clsx(buttonVariants({ variant: "outline" }), props.classNames)}
      onClick={props.onClick}
      to={props.to}
    >
      {props.children}
    </Link>
  );
}
