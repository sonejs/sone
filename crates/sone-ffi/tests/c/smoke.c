/* Renders a fixture through the C ABI and writes it to argv[1].
   Built and run by tests/c_abi.rs, which then diffs it against the CLI. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "sone.h"

static char *read_file(const char *path, size_t *len) {
  FILE *f = fopen(path, "rb");
  if (!f) return NULL;
  fseek(f, 0, SEEK_END);
  long size = ftell(f);
  fseek(f, 0, SEEK_SET);
  char *buf = (char *)malloc((size_t)size + 1);
  if (!buf) { fclose(f); return NULL; }
  if (fread(buf, 1, (size_t)size, f) != (size_t)size) { free(buf); fclose(f); return NULL; }
  buf[size] = '\0';
  fclose(f);
  if (len) *len = (size_t)size;
  return buf;
}

int main(int argc, char **argv) {
  if (argc < 4) {
    fprintf(stderr, "usage: smoke <doc.json> <base-dir> <out.png>\n");
    return 64;
  }

  char *json = read_file(argv[1], NULL);
  if (!json) { fprintf(stderr, "cannot read %s\n", argv[1]); return 3; }

  SoneEngine *engine = sone_engine_new(argv[2]);
  if (!engine) { fprintf(stderr, "engine creation failed\n"); return 4; }

  SoneRenderOptions options;
  memset(&options, 0, sizeof(options));
  options.format = SoneFormat_Png;
  options.density = 2.0f;

  SoneBuffer out;
  SoneStatus status = sone_render_json(engine, json, &options, &out);
  if (status != SoneStatus_Ok) {
    const char *message = sone_engine_last_error(engine);
    fprintf(stderr, "render failed (%d): %s\n", (int)status, message ? message : "unknown");
    sone_engine_free(engine);
    free(json);
    return (int)status;
  }

  FILE *f = fopen(argv[3], "wb");
  if (!f) { fprintf(stderr, "cannot write %s\n", argv[3]); return 3; }
  fwrite(out.data, 1, out.len, f);
  fclose(f);

  printf("sone %s wrote %zu bytes\n", sone_version(), out.len);
  sone_buffer_free(&out);
  sone_engine_free(engine);
  free(json);
  return 0;
}
