#include <iostream>
#include <fstream>
#include <string>
#include <vector>
using namespace std;

const string t = "ABCACCABCB";
const string p = "AB";
int n, m; // size of t (resp. p)
const int k = 0; // max n of mistmaches
vector<int> A_old(k + 1, 0), A_new(k + 1, 0);

// Returns the length of the longest common prefix between suffixes
// p_i...p_m-1 and p_j... p_m-1
// NOTE: not efficient atm
int LCA(int i, int j) {
  int prefix_size = 0;
  while (i < m && j < m && p[i] == p[j]) {
    i++; j++; // dangerous?
    prefix_size++;
  }
  cout << "i: " << i << " j: " << j << " ps: " << prefix_size << endl;
  return prefix_size;
}

int main() {
  n = t.size(), m = p.size();
  int i_old = 0, j = 0, S = 0;
  vector <int> matches;

  for (int i_new = 0; i_new < n - m; i_new++) {
    int q = 0;

    if (i_new < j) {
      // MERGE algorithm
      int s = 1, i = i_new;
      while ((i <= A_old[S]) && (q <= k)) {
        int l = LCA(i - i_new, i - i_old);

        // (1)
        if (i + l < A_old[s]) {
          q++; A_new[q] = i + l; i += l + 1; // (N)
        // (2)
        } else if (i + l == A_old[s]) {
          if (t[A_old[s]] != p[i - i_new]) {
            q++;
            A_new[q] = A_old[s];
          }
          i = A_old[s] + 1; s++;
        // (3)
        } else {
          q++;
          A_new[q] = A_old[s];
          i = A_old[s] + 1; s++;
        }
      }
      while (q <= k) {
        int l = LCA(i - i_new, i - i_old);
        if (i + l > j) break;
        q++; A_new[q] = i + l; i += l + 1; // (N)
      } // end merge

      // DEBUG
      cout << "Merge:  ";
      cout << "s: " << s << " q: " << q << " i:     " << i << " A_new: ";
      for (int x : A_new) cout << x << ' ';
      cout << endl;
    }

    if (q < k+1) {
      // EXTEND algorithm
      while ((q < k + 1) && (j - i_new < m)) {
        j++;
        // mismatch?
        if (t[j] != p[j - i_new]) {
          q++;
          A_new[q] = j;
        }
      } // end extend

      i_old = i_new; A_old = A_new; S = q;

      // DEBUG
      cout << "Extend: ";
      cout << "j: " << j << " q: " << q << " i_old: " << i_old << " A_new: ";
      for (int x : A_new) cout << x << ' ';
      cout << endl;
    }
    // Found match? 
    if (q < k+1) {
      cout << "Found match: ";
      cout << "i_new: " << i_new << " q: " << q << " A_old: " << ' ';
      for (int x : A_old) cout << x << ' '; 
      cout << endl;
      matches.push_back(i_new);
    }
  }

  for (int m : matches) cout << m << ' ';
  cout << endl;

  return 0;
}


