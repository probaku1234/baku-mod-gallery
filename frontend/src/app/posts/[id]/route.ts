import { authOptions } from "@/app/util/auth";
import { generateJwtToken } from "@/app/util/jwt";
import { getServerSession } from "next-auth";
import { NextResponse } from "next/server";

// if (process.env.NODE_ENV === "development") {
//   export const revalidate = 0;
// }
// export const revalidate = 0;

export async function GET(
  request: Request,
  { params }: { params: { id: string } }
) {
  try {
    const uri = `${process.env.API_HOST!}/api/posts/${params.id}`;
    const res = await fetch(uri, {
      method: "POST",
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
    return NextResponse.error();
  }
}

export async function DELETE(
  request: Request,
  { params }: { params: { id: string } }
) {
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

    const uri = `${process.env.API_HOST!}/api/posts/${params.id}`;
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

    if (!res.ok) {
      throw new Error(`${res.status} ${res.statusText}`);
    }
    
    return NextResponse.json({ status: 200 });
  } catch (error) {
    console.error(error);
    return NextResponse.json({ error }, { status: 500 });
  }
}

export async function PUT(
  request: Request,
  { params }: { params: { id: string } }
) {
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
    console.log(requestBody);
    const uri = `${process.env.API_HOST!}/api/posts/${params.id}`;
    const res = await fetch(uri, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(requestBody),
      next: {
        tags: ["posts"],
      },
    });

    if (!res.ok) {
      throw new Error(`${res.status} ${res.statusText}`);
    }

    return NextResponse.json({ status: 200 });
  } catch (error) {
    console.error(error);
    return NextResponse.json({ error }, { status: 500 });
  }
}
