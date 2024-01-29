export {};

declare global {
  interface IPost {
    _id: {
      $oid: string;
    };
    title: string;
    images_url: string[];
    file_url: string;
    mod_type: string;
    created_at: Date;
    updated_at: Date;
  }

  interface IJWTPayload {
    name: string;
    role: string;
  }
}
