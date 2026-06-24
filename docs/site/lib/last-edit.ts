import { getGithubLastEdit } from 'fumadocs-core/content/github';
import { OWNER, REPO, SHA, repoRelativePath } from '@/lib/repo';

interface PageFile {
  absolutePath?: string;
  path: string;
}

// A token raises GitHub's 60 req/hr unauthenticated limit. The docs build makes
// one commits API call per page (per locale), so an unauthenticated build would
// hit the limit. CI exposes GITHUB_TOKEN; locally it is usually unset, in which
// case we skip the lookup entirely so `next dev` / offline builds never fail.
function resolveToken(): string | undefined {
  const raw = process.env.DOCS_GIT_TOKEN ?? process.env.GITHUB_TOKEN;
  return raw ? `Bearer ${raw}` : undefined;
}

// Returns the last edit time of a doc page, or null when unavailable (no token,
// rate limit, network error). Never throws: a docs metadata lookup must not be
// able to fail the static export build.
export async function getLastEdit(page: PageFile): Promise<Date | null> {
  const token = resolveToken();
  if (!token) return null;

  try {
    return await getGithubLastEdit({
      owner: OWNER,
      repo: REPO,
      sha: SHA,
      token,
      path: repoRelativePath(page),
    });
  } catch {
    return null;
  }
}
