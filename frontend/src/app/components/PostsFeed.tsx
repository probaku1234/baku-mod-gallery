import { PostCard } from "./PostCard";

export const PostsFeed = async () => {
  const res = await fetch(`/posts`);

  const { data } = await res.json();

  let posts = data as IPost[];

  return (
    <div className="grid grid-cols-3 gap-4">
      {posts.map((post, index) => (
        <PostCard key={index} post={post} />
      ))}
    </div>
  );
};
