// export const revalidate = 0; // revalidate at most every hour

import { NextResponse } from "next/server";
import { generateJwtToken } from "../util/jwt";
import { authOptions } from "@/app/util/auth";
import { getServerSession } from "next-auth";

export async function GET(request: Request) {
  try {
    const uri = `${process.env.API_HOST!}/api/posts`;
    const res = await fetch(uri, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      next: {
        tags: ["posts"],
      },
    });

    const data = await res.json();

    return NextResponse.json({ data });
  } catch (error) {
    console.error(error);
    return NextResponse.json({ error }, { status: 500 });
  }
}

export async function POST(request: Request) {
  const session = await getServerSession(authOptions);

  // Check if the user is authenticated
  if (!session) {
    return new Response(null, { status: 401 }); // User is not authenticated
  }

  // Check if the user has the 'admin' role
  if (session.user.role !== "admin") {
    return new Response(null, { status: 403 }); // User is authenticated but does not have the right permissions
  }

  try {
    const token = generateJwtToken({
      name: session.user.name!,
      role: session.user.role!,
    });
    const requestBody = await request.json();

    const uri = `${process.env.API_HOST!}/api/posts/create`;
    const res = await fetch(uri, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(requestBody),
      next: {
        tags: ["posts"],
      },
    });

    const data = await res.json();

    return NextResponse.json({ status: 200 });
  } catch (error) {
    console.error(error);
    return NextResponse.json({ error }, { status: 500 });
  }
}

export async function DELETE(request: Request) {
  const session = await getServerSession(authOptions);

  // Check if the user is authenticated
  if (!session) {
    return new Response(null, { status: 401 }); // User is not authenticated
  }

  // Check if the user has the 'admin' role
  if (session.user.role !== "admin") {
    return new Response(null, { status: 403 }); // User is authenticated but does not have the right permissions
  }

  try {
    const token = generateJwtToken({
      name: session.user.name!,
      role: session.user.role!,
    });

    const uri = `${process.env.API_HOST!}/api/posts/`;
    const res = await fetch(uri, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      next: {
        tags: ["posts"],
      },
    });

    const data = await res.json();

    return NextResponse.json({ status: 200 });
  } catch (error) {
    console.error(error);
    return NextResponse.json({ error }, { status: 500 });
  }
}
