import type { NextRequest, NextFetchEvent } from "next/server";
import { NextResponse } from "next/server";
import { getToken } from "next-auth/jwt";

export async function middleware(req: NextRequest, event: NextFetchEvent) {
  // 로그인 했을 경우에만 존재함 ( "next-auth.session-token" 쿠키가 존재할 때 )
  const token = await getToken({ req });
  const { pathname } = req.nextUrl;

  console.log("middleware");
  // 2022/08/13 - 로그인/회원가입 접근 제한 - by 1-blue
  if (pathname.startsWith("/admin")) {
    console.log(token);
    if (!token) {
      return NextResponse.redirect(new URL("/login", req.url));
    } else if (token.role !== "admin") {
      return NextResponse.redirect(new URL("/", req.url));
    }
  }
}

export const config = {
  matcher: ["/admin",],
};
