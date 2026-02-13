export type TreeNode = {
    name: string;
    path: string;
    type: "folder" | "file";
    children?: TreeNode[];
};
