export const revalidate = 0; // revalidate at most every hour

import { getServerSession } from "next-auth";
import { authOptions } from "../util/auth";
import PostTable from "../components/PostTable";

export default async function Page() {
  const res = await fetch(`${process.env.FRONT_HOST!}/posts`);

  const { data } = await res.json();

  let posts = data as IPost[];

  return (
    <div>
      <h1 className="text-3xl font-bold text-center text-6xl">Admin Panel</h1>
      <div className="relative overflow-x-auto shadow-md sm:rounded-lg">
        <PostTable posts={posts} />
      </div>
    </div>
  );
}
