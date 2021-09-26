// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Hdfs Utility

use crate::err::HdfsErr;
use crate::hdfs::HdfsFs;
use crate::native::{hdfsCopy, hdfsMove};
use std::ffi::CString;

/// Hdfs Utility
pub struct HdfsUtil;

impl HdfsUtil {
    /// Copy file from one filesystem to another.
    ///
    /// #### Params
    /// * ```srcFS``` - The handle to source filesystem.
    /// * ```src``` - The path of source file.
    /// * ```dstFS``` - The handle to destination filesystem.
    /// * ```dst``` - The path of destination file.
    pub fn copy(
        src_fs: &HdfsFs,
        src: &str,
        dst_fs: &HdfsFs,
        dst: &str,
    ) -> Result<bool, HdfsErr> {
        let res = unsafe {
            let cstr_src = CString::new(src).unwrap();
            let cstr_dst = CString::new(dst).unwrap();
            hdfsCopy(
                src_fs.raw(),
                cstr_src.as_ptr(),
                dst_fs.raw(),
                cstr_dst.as_ptr(),
            )
        };

        if res == 0 {
            Ok(true)
        } else {
            Err(HdfsErr::Unknown)
        }
    }

    /// Move file from one filesystem to another.
    ///
    /// #### Params
    /// * ```srcFS``` - The handle to source filesystem.
    /// * ```src``` - The path of source file.
    /// * ```dstFS``` - The handle to destination filesystem.
    /// * ```dst``` - The path of destination file.
    pub fn mv(
        src_fs: &HdfsFs,
        src: &str,
        dst_fs: &HdfsFs,
        dst: &str,
    ) -> Result<bool, HdfsErr> {
        let res = unsafe {
            let cstr_src = CString::new(src).unwrap();
            let cstr_dst = CString::new(dst).unwrap();
            hdfsMove(
                src_fs.raw(),
                cstr_src.as_ptr(),
                dst_fs.raw(),
                cstr_dst.as_ptr(),
            )
        };

        if res == 0 {
            Ok(true)
        } else {
            Err(HdfsErr::Unknown)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::hdfs::HdfsFs;
    use crate::test::{get_hdfs, run_hdfs_test};

    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_from_local() {
        // Prepare file source from local file system
        let temp_file = tempfile::Builder::new().tempfile().unwrap();
        let src_path = temp_file.path();
        let src_file_name = src_path.file_name().unwrap().to_str().unwrap();
        let src_file = src_path.to_str().unwrap();

        run_hdfs_test(|dfs| {
            // Source
            let src_fs =
                HdfsFs::new(format!("file://{}", src_path.to_str().unwrap()).as_str())
                    .ok()
                    .unwrap();

            // Destination
            let dst_file = format!("/{}", src_file_name);
            let dst_fs = get_hdfs(dfs);

            assert!(
                HdfsUtil::copy(&src_fs, src_file, &dst_fs, dst_file.as_str())
                    .ok()
                    .unwrap()
            );
            assert!(dst_fs.exist(dst_file.as_str()));
            assert!(Path::new(src_file).exists());

            assert!(dst_fs.delete(dst_file.as_str(), false).ok().unwrap());

            assert!(HdfsUtil::mv(&src_fs, src_file, &dst_fs, dst_file.as_str())
                .ok()
                .unwrap());
            assert!(dst_fs.exist(dst_file.as_str()));
            assert!(!Path::new(src_file).exists());
        });
    }

    #[test]
    fn test_to_local() {
        let file_name = "test.txt";

        let temp_dir = tempdir().unwrap();
        let dst_path = temp_dir.path().join(file_name);
        let dst_file = dst_path.to_str().unwrap();

        run_hdfs_test(|dfs| {
            // Source
            let src_file = format!("/{}", file_name);
            let src_fs = get_hdfs(dfs);
            src_fs.create(src_file.as_str()).ok().unwrap();

            // Destination
            let dst_fs = HdfsFs::new(format!("file://{}", dst_file).as_str())
                .ok()
                .unwrap();

            assert!(
                HdfsUtil::copy(&src_fs, src_file.as_str(), &dst_fs, dst_file)
                    .ok()
                    .unwrap()
            );
            assert!(src_fs.exist(src_file.as_str()));
            assert!(Path::new(dst_file).exists());

            fs::remove_file(dst_file);

            assert!(HdfsUtil::mv(&src_fs, src_file.as_str(), &dst_fs, dst_file)
                .ok()
                .unwrap());
            assert!(!src_fs.exist(src_file.as_str()));
            assert!(Path::new(dst_file).exists());
        });
    }
}
