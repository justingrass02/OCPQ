import { Link } from "react-router-dom";
import { buttonVariants } from "./ui/button";
import { type ReactNode } from "react";
type MenuLinkProps = {
  to: string;
  children: ReactNode;
  onClick?: (ev: React.MouseEvent<HTMLAnchorElement, MouseEvent>) => any;
};
export default function MenuLink(props: MenuLinkProps) {
  return (
    <Link
      className={buttonVariants({ variant: "outline" })}
      onClick={props.onClick}
      to={props.to}
    >
      {props.children}
    </Link>
  );
}
