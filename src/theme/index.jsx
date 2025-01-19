import { extendTheme } from "@yamada-ui/react";
import { styles } from "./styles";

const custumTheme = { 
    styles,
}

export const theme = extendTheme(custumTheme)()
