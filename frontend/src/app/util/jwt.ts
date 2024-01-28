import jwt from "jsonwebtoken";
const { sign, verify } = jwt;

export const generateJwtToken = (payload: IJWTPayload) => {
  return sign(payload, process.env.NEXTAUTH_SECRET!, {
    expiresIn: "1h",
  });
};

export const verifyJwtToken = (token: string) => {
  return verify(token, process.env.NEXTAUTH_SECRET!);
};
