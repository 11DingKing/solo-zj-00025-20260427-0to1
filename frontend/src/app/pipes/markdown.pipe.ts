import { Pipe, PipeTransform } from "@angular/core";
import * as marked from "marked";

@Pipe({
  name: "markdown",
  standalone: true,
})
export class MarkdownPipe implements PipeTransform {
  transform(value: string | undefined | null): string {
    if (!value) {
      return "";
    }

    try {
      return marked.parse(value) as string;
    } catch {
      return value;
    }
  }
}
