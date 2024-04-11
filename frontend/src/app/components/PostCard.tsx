"use client";

import { Card } from "flowbite-react";
import Image from "next/image";
import Link from "next/link";

interface Props {
  post: IPost;
}

export const PostCard = (props: Props) => {
  return (
    <Card
      className=""
      renderImage={() => (
        <Image
          width={0}
          height={0}
          sizes="100vw"
        //   fill
          src={props.post.images_url[0] || "/empty.png"}
          alt="image 1"
          style={{ width: '100%', height: 'auto'}}
        />
      )}
    >
      <Link href={`/posts/detail/${props.post._id.$oid}`}>
        <h5 className="text-2xl font-bold tracking-tight text-gray-900 dark:text-white hover:text-blue-600">
          {props.post.title}
        </h5>
      </Link>

      <p className="font-normal text-gray-700 dark:text-gray-400">
        Here are the biggest enterprise technology acquisitions of 2021 so far,
        in reverse chronological order.
      </p>
    </Card>
  );
};
