import { Link } from "react-router-dom";
import { buttonVariants } from "./ui/button";
import { ReactNode } from "react";
type MenuLinkProps = {
  to: string;
  children: ReactNode;
  onClick?: (ev: React.MouseEvent<HTMLAnchorElement, MouseEvent>) => any;
};
export default function MenuLink(props: MenuLinkProps) {
  return (

    <Link className={buttonVariants({ variant: "outline" })} onClick={props.onClick} to={props.to}>
      {props.children}
    </Link>
    // <NavLink
    //   onClick={(ev) => {
    //     if (props.onClick) {
    //       props.onClick(ev);
    //     }
    //   }}
    //   className={({ isActive, isPending }) =>
    //     `${
    //       isActive
    //         ? "bg-pink-200  hover:bg-pink-400  border-pink-400"
    //         : isPending
    //         ? "bg-purple-200  hover:bg-purple-400  border-purple-400"
    //         : "bg-blue-200  hover:bg-blue-400  border-blue-400"
    //     } block py-2 px-0.5 mx-2 rounded-lg font-semibold text-slate-900 hover:text-white border shadow-md`
    //   }
    //   to={props.to}
    // >
    //   {props.label}
    // </NavLink>
  );
}
