export default async function Page({ params }: { params: { id: string } }) {
  const res = await fetch(`${process.env.FRONT_HOST!}/posts/${params.id}`);

  const { data } = await res.json();
  let post = data as IPost;
  console.log(post);

  // console.log(data.body as IPost);
  return (
    <div>
      My Post: {params.id} {data.created_at}
    </div>
  );
}
