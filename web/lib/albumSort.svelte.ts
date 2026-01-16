export const albumSortState = $state({
    type: 'newest' as 'newest' | 'recent' | 'frequent' | 'random' | 'starred' | 'alphabeticalByName' | 'alphabeticalByArtist' | 'byYear'
});
