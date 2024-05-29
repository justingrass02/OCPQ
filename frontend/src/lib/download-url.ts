export function downloadURL(dataURL: string, name: string) {
  const a = document.createElement("a");
  document.body.appendChild(a);
  a.setAttribute("download", name);
  a.setAttribute("href", dataURL);
  a.click();
  document.body.removeChild(a);
}
