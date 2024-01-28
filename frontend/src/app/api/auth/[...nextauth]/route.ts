import { authOptions } from "@/app/util/auth";
import { NextAuthOptions } from "next-auth";
import NextAuth from "next-auth/next";

// export const authOptions: NextAuthOptions = { 
//   ...authOptions,
// }

const handler = NextAuth(authOptions);

export { handler as GET, handler as POST };
