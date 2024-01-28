"use server";

import { useSession } from "next-auth/react";
import { revalidatePath, revalidateTag } from "next/cache";
import sanitizeHtml from "sanitize-html";
import { generateJwtToken } from "./util/jwt";

// FIXME: allow img tag
export const createOrUpdatePost = async (
  prevState: {
    result: string;
    message: string;
  },
  formData: FormData
) => {
  const rawFormData = Object.fromEntries(formData);
  console.log(rawFormData);
  const postId = (formData.get("postId") || "") as string;
  const title = formData.get("title");
  const content = (formData.get("content") || "") as string;
  const sanitizedContent = sanitizeHtml(content);
  const imagesUrl = [];
  const fileUrl = formData.get("fileUrl");
  const modType = formData.get("modType");

  let i = 0;
  while (true) {
    const imageUrl = formData.get(`imageUrl${i}`);
    if (formData.get(`imageUrl${i}`)) {
      imagesUrl.push(imageUrl);
    } else {
      break;
    }
    i++;
  }

  const requestBody = {
    title,
    content: sanitizedContent,
    imagesUrl,
    fileUrl,
    modType,
  };
  console.log(requestBody);

  try {
    let requestUri;
    let requestOption;

    if (postId === "") {
      requestUri = `${process.env.FRONT_HOST!}/posts`;
      requestOption = {
        method: "POST",
      };
    } else {
      requestUri = `${process.env.FRONT_HOST!}/posts/${postId}`;
      requestOption = {
        method: "PUT",
      };
    }

    const res = await fetch(requestUri, {
      ...requestOption,
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
    });

    if (!res.ok) {
      throw new Error(`${res.status} ${res.statusText}`);
    }

    revalidatePath("/admin");
    revalidateTag("posts");
    return {
      result: "success",
      message: "New post created!",
    };
  } catch (error) {
    console.error(error);
    return {
      result: "fail",
      message: error as string,
    };
  }
};

export const deletePost = async (
  prevState: {
    result: string;
    message: string;
  },
  formData: FormData
) => {
  try {
    const postId = formData.get("postId");
    const res = await fetch(`${process.env.FRONT_HOST!}/posts/${postId}`, {
      method: "DELETE",
    });

    if (!res.ok) {
      throw new Error(`${res.status} ${res.statusText}`);
    }

    revalidatePath("/admin");
    revalidateTag("posts");

    return {
      result: "success",
      message: "Post Deleted!",
    };
  } catch (error) {
    console.error(error);
    return {
      result: "fail",
      message: error as string,
    };
  }
};

export const deleteAllPosts = async (
  prevState: {
    result: string;
    message: string;
  },
  formData: FormData
) => {
  try {
    const res = await fetch(`${process.env.FRONT_HOST!}/posts`, {
      method: "POST",
    });

    if (!res.ok) {
      throw new Error(`${res.status} ${res.statusText}`);
    }

    revalidatePath("/admin");
    revalidateTag("posts");

    return {
      result: "success",
      message: "All Posts Deleted!",
    };
  } catch (error) {
    console.error(error);

    return {
      result: "fail",
      message: error as string,
    };
  }
};
